@tool
extends Polygon2D

signal something_changed()

@export
var border_size := 1:
	set(value):
		border_size = value
		something_changed.emit()
@export
var background_margin := 4:
	set(value):
		background_margin = value
		something_changed.emit()
@export
var size = Vector2i(120, 48):
	set(value):
		size = value
		something_changed.emit()

var border: Polygon2D = null
var background: Polygon2D = null
var label: RichTextLabel = null

func _ready() -> void:
	border = self.get_node("Border")
	background = self.get_node("Background")
	label = self.get_node("Label")
	var callback = func() -> void:
		update_size(self, self.size)
	something_changed.connect(callback)
	something_changed.emit()

func update_size(obj: Polygon2D, size: Vector2i) -> void:
	obj.polygon = PackedVector2Array([
		Vector2(0, 0),
		Vector2(0, size.y),
		size,
		Vector2(size.x, 0),
	])
	if obj == self:
		var border_offset = Vector2i(border_size, border_size)
		update_size(border, size - border_offset * 2)
		border.position = border_offset
		var background_offset = border_offset + Vector2i(background_margin, background_margin)
		update_size(background, size - background_offset * 2)
		background.position = background_offset

func adjust_size():
	label

#//=============================================================================
#// * Max Choice Width
#//=============================================================================
#Window_ChoiceList.prototype.maxChoiceWidth = function() {
  #var maxWidth = 36;
  #var choices = $gameMessage.choices();
  #for (var i = 0; i < choices.length; i++) {
	#var choiceWidth = this.textWidthEx(choices[i]) + this.textPadding() * 2;
	#if (maxWidth < choiceWidth) {
	  #maxWidth = choiceWidth;
	#};
  #};
  #return maxWidth + 28;
#};
