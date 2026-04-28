# mclog

> an **external** service logger for minecraft commands.  

## Logging

The datapack can be found in `mclog`

```mcfunction
function #log:info {msg: "Look! I'm logging!", fn: "example:fn"}
```
It's that easy to log a piece of data externally outside the game.  

`msg` can be any data, a string, a number, an entire object, anything.  
`fn` should be the function in which this log is called, can also be empty.  

You can also of course manually store these in a storage.  
```mcfunction
data modify storage log temp.msg set value 81
data modify storage log temp.fn set value ""
function #log:debug with storage log temp
```

There is also a special field in the `log` storage.  
Anything written to `storage log temp` will be cleared after each log.  

There is 5 levels of logging:  
- Trace
- Debug
- Info
- Warn
- Error

### Contextual Data

Each log also attaches some extra data other than your `msg` and `fn`.  
These fields are automatically added from the command context.  

- Executing **Dimension**  
- Executing **Position** and **Rotation**  
- Current **Game Tick**
- Entity Data  
    - **UUID**
    - **Type**
    - Anything inside `.data` in the entity
- Message (**msg**)
- Function (**fn**)

### Disabling

When you want to publish your datapack for others you may not want to include the logging feature.  
Just delete `data/log/tags/function`,  
and rename `data/log/tags/empty_function` > `data/log/tags/function`.  
This replaces all tagged functions *(which you should be using)* to empty tags.  
Which in process makes all your logging calls useless while not erroring your datapack *(like if you removed the `log` datapack)*.  

### How?

Each log gets sent to your `latest.log` file in your Minecraft instance.  
Below is an example log executed from a strider:

```log
[03:40:03] [Server thread/INFO]: Test log (at BlockPos{x=0, y=0, z=0}): {dimension:"minecraft:the_nether",entity:{data:{},type:"minecraft:strider",uuid:[I;-256617240,730549736,-1446282306,-1252097036]},function:"",level:"info",message:8,pos:[-226.51687942973177d,31.5d,-207.3884943716344d],rotation:[194.03525f,0.0f],tick:168195}
```

The above example log might reveal how we are logging data to outside the game if you look closely.  
We use `test_block` in the `log` mode to insert our own message and activating it with redstone.  
This is done in a custom dimension as to not collide with anything made by the consumer.   

```mcfunction
# log:zzz/place

$setblock 0 0 0 test_block{message:'$(out)', mode:"log"}
setblock 0 1 0 redstone_block
```

This specifically is the sauce of how this all works,  
the rest is just adding contextual data, having tagged functions and handling the dimension.  

Any amount of logs can be sent in the same game tick and will thus share the same `tick` field.  

## Capturing

This project also contains a Rust library and binary that captures & extracts the logs.  
Optionally outputting it via `tracing`.  

This can be found in `mclog_capture`.   

**TODO**