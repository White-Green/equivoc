# frozen_string_literal: true

require "./equivoc"

image = load_image("test.png")

e_for [image.width, image.height] do |x, y|
  p = image.pixel(x, y)
  image.write_pixel(x, y, p)
end

write_image(image, "test2.png")
