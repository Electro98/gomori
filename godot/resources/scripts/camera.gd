extends Camera2D

var player: Player = null
var half_screen = Vector2(0, 0)

func _ready() -> void:
	player = self.get_node("../Player")
	half_screen = get_screen_center_position()

func _process(_delta: float) -> void:
	self.position = player.position + Vector2(16, 16) - half_screen
