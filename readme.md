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
- Log **Level**

### Disabling

When you want to publish your datapack for others you may not want to include the logging feature.  
Just delete `data/log/tags/function`,  
and rename `data/log/tags/empty_function` > `data/log/tags/function`.  
This replaces all tagged functions *(which you should be using)* to empty tags.  
Which in process makes all your logging calls useless while not erroring your datapack *(like if you removed the `log` datapack)*.  

Optionally you could `replace` the tags in a higher priority datapack.  

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
Optionally outputting it via [`tracing`](https://github.com/tokio-rs/tracing).  

This can be found in `mclog_capture`.   
You can download it here: **[Releases](https://github.com/VilleOlof/mclog/releases)**

### Binary

If you don't want to deal with using the library and just want to capture the logs in a nice way.  
This makes it super easy.  

This binary; `mclog`, captures all logs and sends them to:  
- `Stdout`: Your console window
- `mclog.log`: A log file if all captured logs
- `mclog.json`: A JSON structured version of `mclog.log`

The binary has a few ways to search for your `latest.log` file: *(Order of priority)*
- CLI Argument  
    Example: `mclog.exe "../../latest.log"`
- Environment Variable  
    Any path inside `LATEST_LOG` is used.  
    These can be in your system or in a `.env` file in the same folder.  
    ```sh
    # .env
    LATEST_LOG = "F:/.minecraft/logs/latest.log"
    ```
- Current Folder  
    If the `latest.log` is in the same folder as the binary it will use that.  
- Folder traversing  
    If any parent folders from the binary is a `.minecraft` folder,  
    it will go from there and into `.minecraft/logs/latest.log` and use that.  

All of this means that you can simply put the capturing binary inside your datapack folder.  
Run it and it will capture all logs from your Minecraft instance correctly. 

#### Build

Or build it yourself with [Rust](https://rust-lang.org/learn/get-started/)

```sh
cargo build --release
```

### Library

If you want to customize your logs or send it to a service or anything else.  
You'd want to use the library and let it just capture the logs for you to handle.  

Theres 2 ways to use the library depending on if you want to use `tracing` or not.  

#### `mclog_capture::log_with_tracing`
Enabled by default with the `tracing` feature flag.  

This function does *everything* for you, capturing, parsing and logging to tracing.  
Which means you only need to customize your tracing subscribers and call this function.  

```rust
// example which captures logs for 10 seconds with tracing
let sub = tracing_subscriber::FmtSubscriber::builder()
    .with_max_level(Level::TRACE)
    .finish();
tracing::subscriber::set_global_default(sub).unwrap();

let _handle = log_with_tracing("latest.log", WatchConfig::default());
sleep(Duration::from_secs(10));
exit(0);
```

This is how the `mclog` binary works, just a pre-customized tracing with this function.  

#### `mclog_capture::log`

This skips any use of `tracing`.  
It does all the capturing and calls one of your functions on every log captured.  

```rust
let _handle = log(
    "latest.log",
    WatchConfig::default(), 
    |log| {
        println!("{log:?}");
        Ok(())
    }
);
```

#### `mclog_capture::parse_log_line`

This is as barebones as you get, this *only* parses a line from the log file into structured data.  

```rust
let line = r#"[22:03:52] [Server thread/INFO]: Test log (at BlockPos{x=0, y=0, z=0}): { ... }"#;

// second argument is the identifier after `[Server thread/INFO]:`
let log = parse_log_line(line, "Test log")?.unwrap();
```