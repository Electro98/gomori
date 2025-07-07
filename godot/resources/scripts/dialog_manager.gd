extends DialogManager

var main_box: Node2D
var main_box_text: RichTextLabel
var name_box: Node2D
var name_box_text: RichTextLabel

# Text animation
var stops: Array[int] = []
var current_text_progress = 0
var current_stop = 0
var text_anim_running = false
@export_range(1.0, 20.0, 0.1)
var animation_speed: float = 13.8

func _ready_script() -> void:
	main_box = get_node("../MainBox")
	main_box_text = get_node("../MainBox/Label")
	name_box = get_node("../NameBox")
	name_box_text = get_node("../NameBox/Label")

func _process(delta: float) -> void:
	process_text_animation(delta)

func process_text_animation(delta: float):
	if not text_anim_running or stops.is_empty() or current_text_progress >= stops.back():
		return
	var new_stop = stops[current_stop]
	current_text_progress += delta * animation_speed
	if current_text_progress >= new_stop:
		text_anim_running = false
		current_stop += 1
	if main_box_text.visible_characters != floor(current_text_progress):
		main_box_text.visible_characters = floor(current_text_progress)

func _show_text(who: String, text: String, stops: Array[int]) -> void:
	set_speaker(who)
	set_text(text, stops[0] if not stops.is_empty() else 0)
	if not stops.is_empty():
		print("Stops: ", stops)
		current_text_progress = 0
		current_stop = 0
		text_anim_running = true
		self.stops = stops
	# Todo: safe stops if there is

func _show_choice(store_to: String, choice_names: Array[String], choice_texts: Array[String]) -> void:
	pass

func _trigger(what: String) -> void:
	pass

func _end_dialog() -> void:
	print("You smell like dead flowers")
	main_box.hide()
	name_box.hide()

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("action"):
		if is_running():
			step()
		else:
			start("label")

func set_speaker(name: String) -> void:
	if name.is_empty():
		name_box.hide()
	else:
		name_box.show()
		name_box_text.text = name

func set_text(text: String, stop: int = 0) -> void:
	if stop <= 0:
		stop = -1
	main_box.show()
	main_box_text.text = text
	main_box_text.visible_characters = 0
