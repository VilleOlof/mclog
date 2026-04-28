# we do a ton of vehicle trickery to maintain the entity's data
# and use it as a Passenger to gather its ID

execute on vehicle run tag @s add this.vehicle

# based of https://www.reddit.com/r/MinecraftCommands/comments/1irhxmr/get_entity_typeid/
tag @s add this
execute at @s summon area_effect_cloud run tag @s add this.parent
# we no wan se dis
data merge entity @n[tag=this.parent] {\
    Duration:0,\
    Radius:0,\
    WaitTime:0,\
    Age:1,\
    custom_particle:{\
        type:"block",\
        block_state:"air"\
    }\
}
execute at @s run ride @n[tag=this] mount @n[tag=this.parent]
tag @s remove this

execute on vehicle run data modify storage log out.entity.type set from entity @s Passengers[0].id

ride @s mount @n[tag=this.vehicle]
execute on vehicle run tag @s remove this.vehicle