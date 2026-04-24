# frozen_string_literal: true

require "json"

class Instruction
  def to_json(json_state = nil)
    h = {tag: self.class.name.to_sym}
    instance_variables.each do |member|
      member_name = member.to_s.sub!(/^@/, "")
      h[member_name] = instance_variable_get(member)
    end
    JSON.generate(h, json_state)
  end
end

class Function
  def initialize(name, args, instructions)
    @name = name
    @args = args
    @instructions = instructions
  end

  def to_json(json_state = nil)
    h = {
      name: @name,
      args: @args,
      instructions: @instructions
    }
    JSON.generate(h, json_state)
  end
end

class If < Instruction
  def initialize(variables, condition, then_instructions, else_instructions)
    @variables = variables
    @condition = condition
    @then_instructions = then_instructions
    @else_instructions = else_instructions
  end
end

class For < Instruction
  def initialize(variable_updates, loop_count, loop_index, instructions)
    @variable_updates = variable_updates
    @loop_count = loop_count
    @loop_index = loop_index
    @instructions = instructions
  end
end

class Loop < Instruction
  def initialize(variable_updates, instructions)
    @variable_updates = variable_updates
    @instructions = instructions
  end
end

class Break < Instruction; end

class Continue < Instruction; end

class Return < Instruction
  def initialize(value)
    @value = value
  end
end

class CallFunction < Instruction
  def initialize(out, name, args)
    @out = out
    @name = name
    @args = args
  end
end

class LoadIntegerConst < Instruction
  def initialize(out, value)
    @out = out
    @value = value
  end
end

class LoadFloatConst < Instruction
  def initialize(out, value)
    @out = out
    @value = value
  end
end

class LoadStringConst < Instruction
  def initialize(out, value)
    @out = out
    @value = value
  end
end

class LoadBooleanConst < Instruction
  def initialize(out, value)
    @out = out
    @value = value
  end
end

class Add < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class Sub < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class Mul < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class Div < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class Mod < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class Equals < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class NotEquals < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class LessThan < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class LessThanOrEquals < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class GreaterThan < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class GreaterThanOrEquals < Instruction
  def initialize(out, lhs, rhs)
    @out = out
    @lhs = lhs
    @rhs = rhs
  end
end

class LoadImage < Instruction
  def initialize(out, path)
    @out = out
    @path = path
  end
end

class WriteImage < Instruction
  def initialize(image, path)
    @image = image
    @path = path
  end
end

class ImageWidth < Instruction
  def initialize(out, image)
    @out = out
    @image = image
  end
end

class ImageHeight < Instruction
  def initialize(out, image)
    @out = out
    @image = image
  end
end

class ReadImagePixel < Instruction
  def initialize(out, image, x, y)
    @out = out
    @image = image
    @x = x
    @y = y
  end
end

class WriteImagePixel < Instruction
  def initialize(image, x, y, pixel)
    @image = image
    @x = x
    @y = y
    @pixel = pixel
  end
end
