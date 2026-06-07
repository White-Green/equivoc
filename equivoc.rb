# frozen_string_literal: true

require "open3"

require_relative "equivoc_frontend_rb/control"
require_relative "equivoc_frontend_rb/instruction"
require_relative "equivoc_frontend_rb/instruction_collector"
require_relative "equivoc_frontend_rb/lib"
require_relative "equivoc_frontend_rb/variable"

class Object
  def self.method_added(name)
    super
    return unless private_method_defined? name

    m = begin
      method name
    rescue NameError
      return
    end
    return unless (source_location = m.source_location) && source_location[0] == Process.argv0

    define_method name do |*args|
      v = Variable.new
      InstructionCollector.instructions << CallFunction.new(v, name, args.map { |arg| Variable.from(arg) })
      v
    end
    private name

    InstructionCollector.functions << [name, m]
  end
end

Kernel.at_exit do
  functions = InstructionCollector.functions.map do |f|
    name, m = f
    args = Array.new(m.arity) { Variable.new }
    ret = nil
    method_instructions = InstructionCollector.stack_instructions do
      ret = m.call(*args)
    end
    method_instructions << Return.new(Variable.from(ret)) unless ret.nil?
    Function.new(name, args, method_instructions)
  end
  obj = {
    functions: functions,
    main_instructions: InstructionCollector.instructions
  }
  Open3.popen3({"RUSTFLAGS" => "-A warnings"}, "cargo run --package=equivoc_cli --release -- --read-frontend-ir-from-stdin", chdir: __dir__) do |i, o, e, wait_thr|
    t1 = Thread.new do
      o.each { |line| $stdout.puts line }
    end
    t2 = Thread.new do
      e.each { |line| warn line }
    end
    i.puts JSON.dump(obj)
    i.close
    wait_thr.join
    t1.join
    t2.join
  end
end
