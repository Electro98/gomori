extends Node

var static_objects

func is_solid(tile_pos: Vector2i) -> bool:
	return static_objects.is_solid(tile_pos)

func is_interactable(tile_pos: Vector2i) -> bool:
	var object: InteractiveObject = static_objects.get_object(tile_pos)
	if object != null:
		return true
	return false

func _ready() -> void:
	static_objects = get_node("/root/Scene/StaticObjects")
