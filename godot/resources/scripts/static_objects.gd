@tool
extends Node2D

var tile_objects: Array[InteractiveObject] = []

func is_solid(tile_pos: Vector2i) -> bool:
	for child in tile_objects:
		if extract_pos(child) != tile_pos:
			continue
		return child.is_solid
	return false

func get_object(tile_pos: Vector2i) -> InteractiveObject:
	for child in tile_objects:
		if extract_pos(child) != tile_pos:
			continue
		return child
	return null

func extract_pos(child: Node) -> Vector2i:
	var pos = child.get("tile_pos")
	if pos == null:
		pos = child.get("start_pos")
	return pos

func convert_pos(pos: Vector2i) -> Vector2:
	return pos * 32

func refresh_children() -> void:
	tile_objects = []
	for child: InteractiveObject in self.get_children():
		if extract_pos(child) == null:
			continue
		if Engine.is_editor_hint():
			child.centered = false
		tile_objects.append(child)

func _ready() -> void:
	refresh_children()
	var refresh = func(_arg):
		refresh_children()
	if Engine.is_editor_hint():
		child_entered_tree.connect(refresh)
		child_exiting_tree.connect(refresh)

func _process(_delta: float) -> void:
	if not Engine.is_editor_hint():
		return
	for child in tile_objects:
		var tile_pos = extract_pos(child)
		if child.position != convert_pos(tile_pos):
			child.position = convert_pos(tile_pos)
