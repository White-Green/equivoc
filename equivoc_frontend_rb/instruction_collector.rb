# frozen_string_literal: true

module InstructionCollector
  @@instructions = []
  @@functions = []

  def self.instructions
    @@instructions
  end

  def self.functions
    @@functions
  end

  def self.stack_instructions
    raise "block required" unless block_given?

    instructions = @@instructions
    @@instructions = []
    yield
    ret = @@instructions
    @@instructions = instructions
    ret
  end
end
