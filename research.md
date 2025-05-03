## Websocket Initialization code
```
    // Connect to the websocket
     const socket = await new Promise((resolve, reject) => {
       const socket = io.connect({
         path: foundry.utils.getRoute("socket.io"),
         transports: ["websocket"],    // Require websocket transport instead of XHR polling
         upgrade: false,               // Prevent "upgrading" to websocket since it is enforced
         reconnection: true,           // Automatically reconnect
         reconnectionDelay: 500,       // Time before reconnection is attempted
         reconnectionAttempts: 10,     // Maximum reconnection attempts
         reconnectionDelayMax: 500,    // The maximum delay between reconnection attempts
         query: {session: sessionId},  // Pass session info
         cookie: false
       });

       // Confirm successful session creation
       socket.on("session", response => {
         socket.session = response;
         const id = response.sessionId;
         if ( !id || (sessionId && (sessionId !== id)) ) return foundry.utils.debouncedReload();
         console.log(`${vtt} | Connected to server socket using session ${id}`);
         resolve(socket);
       });

       // Fail to establish an initial connection
       socket.on("connectTimeout", () => {
         reject(new Error("Failed to establish a socket connection within allowed timeout."));
       });
       socket.on("connectError", err => reject(err));
     });

     // Buffer events until the game is ready
     socket.prependAny(Game.#bufferSocketEvents);

     // Disconnection and reconnection attempts
     let disconnectedTime = 0;
     socket.on("disconnect", () => {
       disconnectedTime = Date.now();
       ui.notifications.error("You have lost connection to the server, attempting to re-establish.");
     });

     // Reconnect attempt
     socket.io.on("reconnect_attempt", () => {
       const t = Date.now();
       console.log(`${vtt} | Attempting to re-connect: ${((t - disconnectedTime) / 1000).toFixed(2)} seconds`);
     });

     // Reconnect failed
     socket.io.on("reconnect_failed", () => {
       ui.notifications.error(`${vtt} | Server connection lost.`);
       window.location.href = foundry.utils.getRoute("no");
     });

     // Reconnect succeeded
     const reconnectTimeRequireRefresh = 5000;
     socket.io.on("reconnect", () => {
       ui.notifications.info(`${vtt} | Server connection re-established.`);
       if ( (Date.now() - disconnectedTime) >= reconnectTimeRequireRefresh ) {
         foundry.utils.debouncedReload();
       }
     });
     return socket;
    }
```