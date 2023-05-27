# REST interface

Hermes can host a REST API, by default on 3032 port. Following endpoints are defined:

| Purpose            | Endpoint        | Type   | Parameters in URI                                    | Reponse                                     |
|--------------------|-----------------|--------|------------------------------------------------------|---------------------------------------------| 
| Get key            | /db             | GET    | In URI: key=_key_                                    | Json string                                 |
| Set key            | /db             | POST   | Json body: { "key" : _"key"_, "value" : _"value"_ }  | Empty                                       |
| Remove key or path | /db             | DELETE | In URI: key=_key_&kind=record or key=_key_&kind=path | Empty                                       |
| List keys          | /db_list        | GET    | In URI: key=_key_                                    | Json string array                           |
| Trigger hook       | /trigger        | POST   | Json body: { "key" : _"key"_, "value" : _"value"_ }  | Empty                                       |
| Get hook           | /hook           | GET    | In URI: key=_key_                                    | Json { prefix : _prefix_, value : _value_ } |
| Set hook           | /hook           | POST   | Json: { "key" : _"key"_, "value" : _"value"_ }       | Empty                                       |
| Remove hook        | /hook           | DELETE | In URI: key=_prefix_&value=_link_                    | Empty                                       |
| List hooks         | /hook_list      | GET    | In URI: key=_prefix_                                 | List of Hook Json                           |
| Suspend log        | /logger/suspend | POST   | None                                                 | Empty                                       |
| Resume log         | /logger/resume  | POST   | None                                                 | Empty                                       |


