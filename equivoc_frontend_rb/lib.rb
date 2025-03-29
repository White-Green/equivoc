# frozen_string_literal: true
require_relative 'variable'

class Image < Variable
  def width
    out = Variable.new
    InstructionCollector.instructions << ImageWidth.new(out, self)
    out
  end

  def height
    out = Variable.new
    InstructionCollector.instructions << ImageHeight.new(out, self)
    out
  end

  def pixel(x, y)
    out = Variable.new
    InstructionCollector.instructions << ReadImagePixel.new(out, self, x, y)
    out
  end

  def write_pixel(x, y, pixel)
    InstructionCollector.instructions << WriteImagePixel.new(self, x, y, pixel)
  end
end

def load_image(path)
  path = Variable.from(path)
  output = Image.new
  InstructionCollector.instructions << LoadImage.new(output, path)
  output
end

def write_image(image, path)
  image = Variable.from(image)
  path = Variable.from(path)
  InstructionCollector.instructions << WriteImage.new(image, path)
end
