# frozen_string_literal: true

require "./equivoc"

image = load_image("test.png")

e_for image.width do |x|
  e_for image.height do |y|
    p = image.pixel(x, y)
    image.write_pixel(x, y, p)
  end
end

write_image(image, "test2.png")
