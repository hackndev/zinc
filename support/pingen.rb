#!/usr/bin/env ruby

require 'rly'
require 'erubis'

class Map
  attr_reader :prelude, :ports

  def initialize(prelude, ports)
    @prelude = prelude
    @ports = ports
  end
end

class Port
  attr_reader :name, :pins

  def initialize(name, pins)
    @name = name
    @pins = pins
  end
end

class Pin
  attr_reader :index, :functions

  def initialize(index, functions)
    @index = index
    @functions = functions
  end
end

class Function
  attr_reader :name, :attributes

  def initialize(name, attributes)
    @name = name
    @attributes = attributes
  end
end

class PinLex < Rly::Lex
  literals ';*{}[]'
  ignore " \t\n"
  token :PORT, /port/
  token :ID, /\w+/
  token :PRELUDE, /^!.*$/ do |t|
    t.value = t.value[1..-1].strip
    t
  end
end

class PinParse < Rly::Yacc
  def next_pin_index
    @current_pin_index ||= 0
    idx = @current_pin_index
    @current_pin_index += 1
    idx
  end

  rule 'map : prelude ports
            | ports' do |ret, p1, p2|
    prelude = p2 ? p1.value : []
    ports = p2 ? p2.value : p1.value
    ret.value = Map.new(prelude, ports)
  end

  rule 'prelude : PRELUDE
                | prelude PRELUDE' do |ret, p1, p2|
    ret.value = if p2
      p1.value + [p2.value]
    else
      [p1.value]
    end
  end

  rule 'ports : port
              | ports port' do |ret, p1, p2|
    ret.value = if p2
      @current_pin_index = 0
      p1.value + [p2.value]
    else
      @current_pin_index = 0
      [p1.value]
    end
  end

  rule 'port : PORT ID "{" pins "}"' do |ret, _, pid, _, pins, _|
    ret.value = Port.new(pid.value, pins.value.compact)
  end

  rule 'pins : pin
             | pins pin' do |ret, p1, p2|
    ret.value = if p2
      p1.value + [p2.value]
    else
      [p1.value]
    end
  end

  rule 'pin : functions ";"' do |ret, f|
    index = next_pin_index
    unless f.value.compact.empty?
      ret.value = Pin.new(index, f.value)
    else
      ret.value = Pin.new(index, [])
    end
  end

  rule 'functions : function
                  | functions function' do |ret, f1, f2|
    ret.value = if f2
      f1.value + [f2.value]
    else
      [f1.value]
    end
  end

  rule 'function : ID
                 | "*"
                 | ID "[" attributes "]"' do |ret, fid, _, attrs, _|
    unless fid.type == '*'
      ret.value = Function.new(fid.value, attrs || [])
    end
  end

  rule 'attributes : attribute
                   | attributes attribute' do |ret, f1, f2|
    ret.value = if f2
      f1.value + [f2.value]
    else
      [f1.value]
    end
  end

  rule 'attribute : ID' do |ret, k|
    ret.value = k.value
  end
end

def gen_lpc(src)
  map = PinParse.new(PinLex.new).parse(src)
  eruby = Erubis::Eruby.new(LPC_TEMPLATE)
  eruby.result(ports: map.ports, prelude: map.prelude).gsub(/^\s+\n/, '')
end

def unwrap_traits(attrs)
  traits = {
    gpio: false,
    uart: false,
    uart_kind: nil,
    uart_index: 0,
  }
  case attrs.shift
    when 'gpio'
      traits[:gpio] = true
    when 'uart'
      traits[:uart] = true
      traits[:uart_kind] = attrs.shift
      traits[:uart_index] = attrs.shift
  end

  traits.map do |k,v|
    v = 'None' if v.nil?
    "#{k}: #{v}"
  end.join(', ')
end

LPC_TEMPLATE = <<-EOF
<%= prelude.join("\n") %>

use std::collections::hashmap::HashMap;

type PinDef = HashMap<String, uint>;
type PinsDef = Vec<Option<PinDef>>;

pub fn port_def() -> HashMap<String, PinsDef> {
  let mut h = HashMap::new();

  <% for port in ports %>
  {
    let mut pins = Vec::new();
    <% for pin in port.pins %>
    {
      <% if pin.functions.length > 0 %>
      let mut pin = HashMap::new();
      <% pin.functions.each_with_index do |fn, i| %><% unless fn.nil? %>
      pin.insert("<%= fn.name.downcase %>".to_str(), <%= i+1 %>);
      <% end %><% end %>
      pins.push(Some(pin));
      <% else %>
      pins.push(None);
      <% end %>
    }
    <% end %>
    h.insert("<%= port.name %>".to_str(), pins);
  }
  <% end %>

  h
}
EOF

f_in = ARGV[0]
f_out = ARGV[1]

open(f_out, 'w') do |f|
  src = open(f_in).read
  output = gen_lpc(src)
  f.write(output)
end
