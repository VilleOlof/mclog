data modify storage log out.level set value "info"
$data modify storage log out.message set value $(msg)
$data modify storage log out.function set value '$(fn)'

function log:zzz/raw