// Live-reload client. Connects to the server's SSE endpoint (only active when
// `serve --watch` is running); reloads the page when a rebuild completes.
// Harmless no-op when the endpoint is absent.
(function () {
    if (!window.EventSource) return;
    try {
        var es = new EventSource("/__reload");
        es.addEventListener("reload", function () {
            window.location.reload();
        });
        es.onerror = function () {
            // Endpoint unavailable (not in watch mode); stop retrying.
            es.close();
        };
    } catch (e) {
        /* ignore */
    }
})();
