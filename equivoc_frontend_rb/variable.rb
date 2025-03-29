# frozen_string_literal: true

require "json"
require_relative "instruction"
require_relative "instruction_collector"

class Variable
  @@max_id = 0

  def initialize
    @id = @@max_id += 1
  end

  def to_json(json_state = nil)
    JSON.generate(@id, json_state)
  end

  def self.from(value)
    return value if value.is_a? Variable

    v = Variable.new
    inst = case value
    when Integer
      LoadIntegerConst.new(v, value)
    when Float
      LoadFloatConst.new(v, value)
    when String
      LoadStringConst.new(v, value)
    when TrueClass
      LoadBooleanConst.new(v, true)
    when FalseClass
      LoadBooleanConst.new(v, false)
    else
      raise "Unknown value type"
    end
    InstructionCollector.instructions << inst
    v
  end

  def self.add(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Add.new(v, lhs, rhs)
    v
  end

  def self.sub(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Sub.new(v, lhs, rhs)
    v
  end

  def self.mul(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Mul.new(v, lhs, rhs)
    v
  end

  def self.div(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Div.new(v, lhs, rhs)
    v
  end

  def self.mod(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Mod.new(v, lhs, rhs)
    v
  end

  def self.equals(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << Equals.new(v, lhs, rhs)
    v
  end

  def self.not_equals(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << NotEquals.new(v, lhs, rhs)
    v
  end

  def self.less_than(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << LessThan.new(v, lhs, rhs)
    v
  end

  def self.less_than_or_equals(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << LessThanOrEquals.new(v, lhs, rhs)
    v
  end

  def self.greater_than(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << GreaterThan.new(v, lhs, rhs)
    v
  end

  def self.greater_than_or_equals(lhs, rhs)
    lhs = Variable.from(lhs)
    rhs = Variable.from(rhs)
    v = Variable.new
    InstructionCollector.instructions << GreaterThanOrEquals.new(v, lhs, rhs)
    v
  end

  def +(other)
    Variable.add(self, other)
  end

  def -(other)
    Variable.sub(self, other)
  end

  def *(other)
    Variable.mul(self, other)
  end

  def /(other)
    Variable.div(self, other)
  end

  def %(other)
    Variable.mod(self, other)
  end

  def ==(other)
    Variable.equals(self, other)
  end

  def !=(other)
    Variable.not_equals(self, other)
  end

  def <(other)
    Variable.less_than(self, other)
  end

  def <=(other)
    Variable.less_than_or_equals(self, other)
  end

  def >(other)
    Variable.greater_than(self, other)
  end

  def >=(other)
    Variable.greater_than_or_equals(self, other)
  end
end

class Integer
  alias_method :original_add, :+
  alias_method :original_sub, :-
  alias_method :original_mul, :*
  alias_method :original_div, :/
  alias_method :original_mod, :%
  alias_method :original_equals, :==
  alias_method :original_not_equals, :!=
  alias_method :original_less_than, :<
  alias_method :original_less_than_or_equals, :<=
  alias_method :original_greater_than, :>
  alias_method :original_greater_than_or_equals, :>=

  def +(other)
    return Variable.add(self, other) if other.is_a? Variable

    original_add other
  end

  def -(other)
    return Variable.sub(self, other) if other.is_a? Variable

    original_sub other
  end

  def *(other)
    return Variable.mul(self, other) if other.is_a? Variable

    original_mul other
  end

  def /(other)
    return Variable.div(self, other) if other.is_a? Variable

    original_div other
  end

  def %(other)
    return Variable.mod(self, other) if other.is_a? Variable

    original_mod other
  end

  def ==(other)
    return Variable.equals(self, other) if other.is_a? Variable

    original_equals other
  end

  def !=(other)
    return Variable.not_equals(self, other) if other.is_a? Variable

    original_not_equals other
  end

  def <(other)
    return Variable.less_than(self, other) if other.is_a? Variable

    original_less_than other
  end

  def <=(other)
    return Variable.less_than_or_equals(self, other) if other.is_a? Variable

    original_less_than_or_equals other
  end

  def >(other)
    return Variable.greater_than(self, other) if other.is_a? Variable

    original_greater_than other
  end

  def >=(other)
    return Variable.greater_than_or_equals(self, other) if other.is_a? Variable

    original_greater_than_or_equals other
  end
end

class Float
  alias_method :original_add, :+
  alias_method :original_sub, :-
  alias_method :original_mul, :*
  alias_method :original_div, :/
  alias_method :original_mod, :%
  alias_method :original_equals, :==
  alias_method :original_not_equals, :!=
  alias_method :original_less_than, :<
  alias_method :original_less_than_or_equals, :<=
  alias_method :original_greater_than, :>
  alias_method :original_greater_than_or_equals, :>=

  def +(other)
    return Variable.add(self, other) if other.is_a? Variable

    original_add other
  end

  def -(other)
    return Variable.sub(self, other) if other.is_a? Variable

    original_sub other
  end

  def *(other)
    return Variable.mul(self, other) if other.is_a? Variable

    original_mul other
  end

  def /(other)
    return Variable.div(self, other) if other.is_a? Variable

    original_div other
  end

  def %(other)
    return Variable.mod(self, other) if other.is_a? Variable

    original_mod other
  end

  def ==(other)
    return Variable.equals(self, other) if other.is_a? Variable

    original_equals other
  end

  def !=(other)
    return Variable.not_equals(self, other) if other.is_a? Variable

    original_not_equals other
  end

  def <(other)
    return Variable.less_than(self, other) if other.is_a? Variable

    original_less_than other
  end

  def <=(other)
    return Variable.less_than_or_equals(self, other) if other.is_a? Variable

    original_less_than_or_equals other
  end

  def >(other)
    return Variable.greater_than(self, other) if other.is_a? Variable

    original_greater_than other
  end

  def >=(other)
    return Variable.greater_than_or_equals(self, other) if other.is_a? Variable

    original_greater_than_or_equals other
  end
end
