# Redis from scratch

This is my rust implementation from following this [guide](https://www.build-redis-from-scratch.dev/en/introduction).

## Todo: Complete the stages in the codecrafters 'Build your own redis' challenge.

### Basic Implementation
- [x] Bind to a port
- Respond to PING
- Respond to multiple PINGs
- Handle concurrent clients
- Implement ECHO command
- [x] Implement SET command & GET command
- Expiry

### RDB Persistence
- RDB file config
- Read a key
- Read a string value
- Read multiple keys
- Read multiple string values
- Read value with expiry

### Replication
- Configure listening port
- The INFO command
- The INFO command on a replica
- Initial Replication ID and Offset
- Send handshake 1/3
- Send handshake 2/3
- Send handshake 3/3
- Receive handshake 1/2
- Receive handshake 2/2
- Empty RDB Transfer
- Single-Replica Command Propagation
- Multi Replica Command Propagation
- Command Processing
- ACKs with no commands
- ACKS with commands
- WAIT with no replicas
- WAIT with no commands
- WAIT with multiple commands

### Streams
- The TYPE command
- Create a stream
- Validating entry IDs
- Partially auto-generated IDs
- Fully auto-generated IDs
- Query entries from stream
- Query with -
- Query with +
- Query single stream using XREAD
- Query multiple streams using XREAD
- Blocking reads
- Blocking reads without timeout
- Blocking reads using $

### Transactions
- The INCR command 1/3
- The INCR command 2/3
- The INCR command 3/3
- The MULTI command
- The EXEC command
- Empty transaction
- Queueing commands
- Executing a transaction
- The DISCARD command
- Failures within transactions
- Multiple transactions
