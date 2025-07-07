extends Player

@export var start_pos = Vector2i(0, 0)
var sprite: AnimatedSprite2D = null
var world
var last_direction = "Down"
const dir2anim = {
	"Up": "walk_up",
	"Left": "walk_left",
	"Down": "walk_down",
	"Right": "walk_right",
}

func _ready() -> void:
	self.setup()
	sprite = self.get_node("Sprite")
	world = self.get_node("../World")
	self.teleport_to(start_pos.x, start_pos.y)

func _process(_delta: float) -> void:
	if not self.is_moving():
		update_movement()
	update_animation()

func update_movement() -> void:
	if GameState.current_state() != GameState.WORLD:
		return
	var x = 0
	var y = 0
	if Input.is_action_pressed("up"):
		y -= 1
	elif Input.is_action_pressed("left"):
		x -= 1
	elif Input.is_action_pressed("down"):
		y += 1
	elif Input.is_action_pressed("right"):
		x += 1
	if x != 0 or y != 0:
		var next_pos = self.tile_pos() + Vector2i(x, y)
		self.move_to(next_pos.x, next_pos.y)
	elif Input.is_action_just_pressed("action"):
		var in_front_pos = self.in_front()
		if world.is_interactable(in_front_pos):
			print("Found in ", in_front_pos.x, ", ", in_front_pos.y, " interactable!")
			if in_front_pos == Vector2i(26, 29):
				get_node("/root/Scene/Dialog/DialogManager").start("label")

func update_animation() -> void:
	var cur_dir = self.direction()
	if last_direction != cur_dir:
		sprite.animation = dir2anim[cur_dir]
		last_direction = cur_dir
	if not self.is_moving() and sprite.is_playing():
		sprite.stop()
	elif self.is_moving() and not sprite.is_playing():
		sprite.play()
