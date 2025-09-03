extends CanvasLayer

@export var health_path: NodePath
var health: Node

func _ready():
    if health_path != NodePath(""):
        health = get_node_or_null(health_path)
    if health and health.has_signal("health_changed"):
        health.health_changed.connect(_on_health_changed)
    if health and health.has_method("get_health_percent"):
        _sync_health_bar()

func _sync_health_bar():
    var hb := $Root/VBox/Health/HealthBar
    if health and hb:
        var p := health.get_health_percent()
        hb.max_value = float(health.max_health)
        hb.value = float(health.current_health)

func _on_health_changed(_amount: int, _type):
    _sync_health_bar()

func log(msg: String):
    var r := $Root/VBox/Messages
    if r:
        r.append_text("\n" + msg)

#EOF