# gRPC inteface

Hermes can host a gRPC interface, by default on 3031 port. Following files is the proto file for interface:
```proto
syntax = "proto3";

option csharp_namespace = "HermesGrpc";

package hermes;

service Hermes {
    rpc Set (Pair) returns (Empty);
    rpc Get (Key) returns (Pair);
    rpc DeleteKey (Key) returns (Empty);
    rpc DeletePath (Key) returns (Empty);
    rpc ListKeys (Key) returns (KeyList);
    rpc SetHook (Pair) returns (Empty);
    rpc GetHook (Key) returns (Hook);
    rpc DeleteHook (Pair) returns (Empty);
    rpc ListHooks (Key) returns (HookCollection);
    rpc SuspendLog (Empty) returns (Empty);
    rpc ResumeLog (Empty) returns (Empty);
}

message Key {
    string key = 1;
}

message Empty {}

message Pair {
    string key = 1;
    string value = 2;
}

message KeyList {
    repeated string keys = 1;
}

message LinkCollection {
    repeated string links = 1;
}

message Hook {
    string prefix = 1;
    LinkCollection links = 2;
}

message HookCollection {
    repeated Hook hooks = 1;
}
```