// Preloaded (node --import) by npm/sdk/method-coverage.mjs into every test process. Wraps the sync channel and the
// async client of the source-mode SDK (npm/tsrs/src) so each API request the upstream tests send to the server is
// appended to $TSRS_METHOD_LOG as `<sync|async>\t<method>\t<ok|error>` lines; batchRequests also logs its inner
// methods. The vendored SDK itself is not modified.
import fs from "node:fs";

const log = process.env.TSRS_METHOD_LOG;
if (log) {
    const src = new URL("../../tsrs/src/api/", import.meta.url);
    const { SyncRpcChannel } = await import(new URL("syncChannel.ts", src).href);
    const { createClient } = await import(new URL("async/client.ts", src).href);
    // #64174 hides RpcClient behind createClient. Construction does not connect or own a subprocess;
    // obtain only its native prototype so injected test transports are never counted as server coverage.
    const asyncPrototype = Object.getPrototypeOf(createClient({ tsserverPath: process.env.TSRS_BINARY }));

    const write = (variant, method, outcome) => fs.appendFileSync(log, `${variant}\t${method}\t${outcome}\n`);
    const inner = (variant, params, outcome) => {
        let parsed = params;
        if (typeof params === "string") {
            try {
                parsed = JSON.parse(params);
            }
            catch {
                return;
            }
        }
        for (const request of parsed?.requests ?? []) write(variant, request.method, outcome);
    };

    for (const name of ["requestSync", "requestBinarySync"]) {
        const original = SyncRpcChannel.prototype[name];
        SyncRpcChannel.prototype[name] = function (method, payload) {
            try {
                const result = original.call(this, method, payload);
                write("sync", method, "ok");
                if (method === "batchRequests" && typeof payload === "string") inner("sync", payload, "ok");
                return result;
            }
            catch (e) {
                write("sync", method, "error");
                if (method === "batchRequests" && typeof payload === "string") inner("sync", payload, "error");
                throw e;
            }
        };
    }

    const originalSend = asyncPrototype.sendRequestWithTiming;
    asyncPrototype.sendRequestWithTiming = async function (requestType, params) {
        try {
            const result = await originalSend.call(this, requestType, params);
            write("async", requestType.method, "ok");
            if (requestType.method === "batchRequests") inner("async", params, "ok");
            return result;
        }
        catch (e) {
            write("async", requestType.method, "error");
            if (requestType.method === "batchRequests") inner("async", params, "error");
            throw e;
        }
    };
}
