# Hermes documents

Document has three major parts, they are discussing the actions
- [Actions](Actions.md)

Following documents describes each interface parameters:
- [Classic TCP interface](Interface_classic.md)
- [gRPC interface](Interface_gRPC.md)
- [REST interface](Interface_REST.md)

As Hermes is inteded to run on back-end server among other APIs and components, and not available directly from front-end applications. For this reason, all interfaces are unsecured to available more speed as possible. If, for some reason, any of these interfaces would be avaiable from front-end, it is handy to put it behind a proxy (e.g.: HAProxy) and setup security there.
