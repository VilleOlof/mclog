# we are still positioned as command ctx
rotate @n[tag=log] ~ ~

# pos & rot
data modify storage log out.pos set from entity @s Pos
data modify storage log out.rotation set from entity @s Rotation

# dimension
data remove storage log out.dimension
data modify storage log out.dimension set from entity @s Dimension
# if the entity doesnt have a Dimensions field, we can only check for vanilla dimensions manually
execute unless data storage log out.dimension run function log:zzz/dimension_check

kill @s