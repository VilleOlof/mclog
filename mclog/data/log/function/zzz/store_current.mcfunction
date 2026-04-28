function log:zzz/default

# tick
execute store result storage log out.tick int 1 run time query gametime

# if we are not an entity (server)
execute unless entity @s run return 1

# uuid of current
data modify storage log out.entity.uuid set from entity @s UUID

# we dont want to mess with the entity's passengers/vechile if they have some
execute unless entity @s[type=player] run function log:zzz/get_type
# this above doesnt work on player soo
execute if entity @s[type=player] run data modify storage log out.entity.type set value "minecraft:player"
# if both the above fail it will default to '?'

# we spawn a temporary marker to get info on the execution position, rotation and other data
# we dont seek the current entity's data, rather where the command is executing
# and we first only wanna execute as the marker, not at
summon marker ~ ~ ~ {Tags:["log"]}
execute as @n[tag=log] run function log:zzz/gather

# entity.data
# if consumer wants more data attached, either send it through "msg" or attach it to .data
# or just modify after this line and add whatever fields you'd want to out.entity.*
execute if data entity @s data run data modify storage log out.entity.data set from entity @s data