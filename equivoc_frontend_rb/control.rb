# frozen_string_literal: true

require_relative "instruction_collector"
require_relative "variable"

def e_if(condition, then_block = nil, else_block = nil, &block)
  unless block.nil?
    raise StandardError.new unless then_block.nil?
    raise StandardError.new unless else_block.nil?
    then_block = block
  end
  condition = Variable.from(condition)
  then_block.binding.local_variables.each do |variable_name|
    then_block.binding.local_variable_set(variable_name, Variable.from(then_block.binding.local_variable_get(variable_name)))
  end
  before_variables = then_block.binding.local_variables.map do |variable_name|
    [variable_name, then_block.binding.local_variable_get(variable_name)]
  end.to_h
  then_instructions = InstructionCollector.stack_instructions do
    then_block.call
    then_block.binding.local_variables.each do |variable_name|
      then_block.binding.local_variable_set(variable_name, Variable.from(then_block.binding.local_variable_get(variable_name)))
    end
  end
  variable_updates = then_block.binding.local_variables.map do |variable_name|
    value = then_block.binding.local_variable_get(variable_name)
    [variable_name, {then: value}] unless value.equal?(before_variables[variable_name])
  end.filter { |pair| pair }.to_h
  else_instructions = []
  unless else_block.nil?
    variable_updates.each do |variable_name, _|
      else_block.binding.local_variable_set(variable_name, before_variables[variable_name])
    end
    else_instructions = InstructionCollector.stack_instructions do
      else_block.call
      else_block.binding.local_variables.each do |variable_name|
        else_block.binding.local_variable_set(variable_name, Variable.from(else_block.binding.local_variable_get(variable_name)))
      end
    end
    else_block.binding.local_variables.each do |variable_name|
      value = else_block.binding.local_variable_get(variable_name)
      next if value.equal? before_variables[variable_name]
      if variable_updates.key? variable_name
        variable_updates[variable_name][:else] = value
      else
        variable_updates[variable_name] = {else: value}
      end
    end
  end
  variable_updates.each do |variable_name, values|
    unless values.key? :then
      values[:then] = before_variables[variable_name]
    end
    unless values.key? :else
      values[:else] = before_variables[variable_name]
    end
    values[:variable] = Variable.new
  end
  variable_updates.each do |variable_name, values|
    then_block.binding.local_variable_set(variable_name, values[:variable])
  end
  variable_updates = variable_updates.values do |value|
    value
  end
  InstructionCollector.instructions << If.new(variable_updates, condition, then_instructions, else_instructions)
end

def e_for(loop_counts, &block)
  loop_counts = [loop_counts] unless loop_counts.instance_of? Array
  loop_counts = loop_counts.map { Variable.from(it) }
  loop_indices = loop_counts.map { Variable.new }

  block.binding.local_variables.each do |variable_name|
    block.binding.local_variable_set(variable_name, Variable.from(block.binding.local_variable_get(variable_name)))
  end
  before_variables = block.binding.local_variables.map do |variable_name|
    [variable_name, block.binding.local_variable_get(variable_name)]
  end.to_h

  block_instruction = InstructionCollector.stack_instructions do
    block.call(*loop_indices)
  end
  variable_updates = block.binding.local_variables.map do |variable_name|
    updated_value = block.binding.local_variable_get(variable_name)
    next [before_variables[variable_name], updated_value] unless before_variables[variable_name].equal? updated_value
  end.filter { |pair| pair }

  InstructionCollector.instructions << For.new(variable_updates, loop_counts, loop_indices, block_instruction)
end

def e_loop(&block)
  block.binding.local_variables.each do |variable_name|
    block.binding.local_variable_set(variable_name, Variable.from(block.binding.local_variable_get(variable_name)))
  end
  before_variables = block.binding.local_variables.map do |variable_name|
    [variable_name, block.binding.local_variable_get(variable_name)]
  end.to_h

  block_instruction = InstructionCollector.stack_instructions do
    block.call
  end
  variable_updates = block.binding.local_variables.map do |variable_name|
    updated_value = block.binding.local_variable_get(variable_name)
    next [before_variables[variable_name], updated_value] unless before_variables[variable_name].equal? updated_value
  end.filter { |pair| pair }

  InstructionCollector.instructions << Loop.new(variable_updates, block_instruction)
end

def e_break
  InstructionCollector.instructions << Break.new
end

def e_continue
  InstructionCollector.instructions << Continue.new
end

def e_return(value = nil)
  value = Variable.from(value) unless value.nil?
  InstructionCollector.instructions << Return.new(value)
end
