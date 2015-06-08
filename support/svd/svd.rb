#!/usr/bin/env ruby

require 'nokogiri'
require 'erubis'

class SVD
  attr_reader :size, :access, :peripherals, :peripherals_map

  def initialize(root)
    @size = root.xpath_text('size').to_i
    @access = root.xpath_text('access')

    @peripherals = []
    @peripherals_map = {}
    root.xpath('peripherals/peripheral').each do |pelem|
      p = Peripheral.new(pelem, self)

      @peripherals << p
      @peripherals_map[p.name] = p
    end
  end
end

class Peripheral
  attr_reader :name, :description, :access, :size, :dim, :registers, :base_address

  def initialize(elem, svd)
    if elem['derivedFrom']
      parent = svd.peripherals_map[elem['derivedFrom']]
      @name = parent.name
      @description = parent.description
      @access = parent.access
      @size = parent.size
      @dim = parent.dim
      @registers = parent.registers.clone
      @base_address = parent.base_address
    end

    @name = elem.xpath_text('name') if elem.xpath_text('name')
    @description = elem.xpath_text('description') if elem.xpath_text('description')
    @access = elem.xpath_text('access') if elem.xpath_text('access')
    @access = svd.access unless @access
    @size = elem.xpath_text('size').to_i if elem.xpath_text('size')
    @size = svd.size unless @size
    @dim = elem.xpath_text('dim').to_i if elem.xpath_text('dim')
    @base_address = elem.xpath_text('baseAddress').to_some_i if elem.xpath_text('baseAddress')

    raise RuntimeError.new("can't derive with registers overriden") if @registers && elem.at_xpath('registers')

    unless @registers
      @registers = []
      @registers_map = {}
      elem.xpath('registers/register').each do |relem|
        rr = if relem.at_xpath('dim')
          Register.make_dim(relem, self)
        else
          [Register.new(relem, self)]
        end

        @registers += rr
        rr.each { |r| @registers_map[r.name] = r }
      end
    end
  end
end

class Register
  attr_reader :name, :description, :access, :size, :offset, :fields, :fields_map
  attr_writer :name, :offset

  def initialize(elem, peripheral)
    if elem['derivedFrom']
      parent = peripheral.registers_map[elem['derivedFrom']]
      @name = parent.name
      @description = parent.description
      @access = parent.access
      @size = parent.size
      @offset = parent.offset
      @fields = parent.fields
    end

    @name = elem.xpath_text('name').downcase if elem.xpath_text('name')
    @description = elem.xpath_text('description') if elem.xpath_text('description')
    @access = elem.xpath_text('access') if elem.xpath_text('access')
    @access = peripheral.access unless @access
    @size = elem.xpath_text('size').to_i if elem.xpath_text('size')
    @size = peripheral.size unless @size
    @offset = elem.xpath_text('addressOffset').to_some_i

    raise RuntimeError.new("can't derive with fields overriden") if @fields && elem.at_xpath('fields')
    unless @fields
      @fields = []
      @fields_map = {}
      elem.xpath('fields/field').each do |felem|
        unless felem.xpath_text('name').downcase == 'reserved'
          f = Field.new(felem, self)
          @fields << f
          @fields_map[f.name] = f
        end
      end
    end
  end

  def self.make_dim(elem, peripheral)
    dim = elem.xpath_text('dim').to_i
    dim_increment = elem.xpath_text('dimIncrement').to_some_i

    # From the SVD XML Schema:
    # <xs:pattern value="[0-9]+\-[0-9]+|[A-Z]-[A-Z]|[_0-9a-zA-Z]+(,\s*[_0-9a-zA-Z]+)+"/>
    dim_index = elem.xpath_text('dimIndex')
    matches = /([0-9]+\-[0-9]+)|([A-Z]-[A-Z])|([_0-9a-zA-Z]+(,\s*[_0-9a-zA-Z]+)+)/.match(dim_index)
    if matches[1] # number range
      index_from, index_to = matches[1].split('-').map { |a| a.to_i }
      raise RuntimeError.new("unsupported dimIndex #{dim_index} with dim #{dim} for #{reg.name}") unless dim == index_to-index_from+1
      indices = *(index_from..index_to)
    elsif matches[3] # comma separated
      indices = dim_index.split('c').map { |a| a.to_i }
    else
      raise RuntimeError("unsupported dimIndex #{dim_index} with dim #{dim} for #{reg.name}")
    end

    regs = []
    ofs = elem.xpath_text("addressOffset").to_some_i
    indices.each do |idx|
      r = Register.new(elem, peripheral)

      r.name = r.name.gsub('%s', idx.to_s)
      r.offset = ofs

      regs << r
      ofs += dim_increment
      idx += 1
    end
    regs
  end
end

class Field
  attr_reader :name, :description, :access, :bits_string, :enums

  def initialize(elem, register)
    if elem['derivedFrom']
      parent = register.fields_map[elem['derivedFrom']]

      @name = parent.name
      @description = parent.description
      @access = parent.access
      @enums = parent.enums
    end

    @name = elem.xpath_text('name').downcase if elem.xpath_text('name')
    @description = elem.xpath_text('description') if elem.xpath_text('description')
    @access = elem.xpath_text('access') if elem.xpath_text('access')
    @access = register.access unless @access

    raise RuntimeError.new("unsupported field schema lsb/msb") if elem.at_xpath('lsb')

    bit_range = elem.xpath_text('bitRange')
    if bit_range
      range = /\[(\d+):(\d+)\]/.match(bit_range)
      @bits_string = if range[1] == range[2]
                       range[1]
                     else
                       "#{range[2]}..#{range[1]}"
                     end
    else
      bit_offset = elem.at_xpath('bitOffset').content.to_i
      bit_width = elem.at_xpath('bitWidth').content.to_i
      bit_end_range = bit_offset + bit_width
      @bits_string = "#{bit_end_range}..#{bit_offset}"
    end

    raise RuntimeError.new("can't derive with enums overriden") if @enums && elem.at_xpath('enumeratedValues')
    unless @enums
      @enums = []
      elem.xpath('enumeratedValues/enumeratedValue').each do |eelem|
        e = Enum.new(eelem, self)
        @enums << e
      end
    end

    @name = "f_#{@name}" if @name == 'match'
  end
end

class Enum
  attr_reader :name, :description, :value

  def initialize(elem, field)
    @name = elem.xpath_text('name')
    @description = elem.xpath_text('description') if elem.xpath_text('description')
    # regex used because freescale prepends a '#' before the actual value
    raw_value = elem.xpath_text('value')
    if raw_value[0] == '#'
      raw_value = raw_value[1..-1]
      value = 0
      raw_value.each_char.with_index { |bit, index| value |= (bit == '1' ? 1 : 0) << index }
      @value = value
    else
      @value = raw_value.to_some_i
    end
    @name = "E_#{@name}" if @name[0] =~ /\d/
  end
end

module Nokogiri
  module XML
    class Element
      def xpath_text(child_name)
        c = self.xpath(child_name)
        if c.empty?
          nil
        else
          c.text.strip.gsub("\n", " ")
        end
      end
    end
  end
end

class String
  def to_some_i
    self['x'] ? to_i(16) : to_i
  end
end

class Fixnum
  def to_hex
    '0x' + self.to_s(16).rjust(2, '0')
  end
end

fn = ARGV[0]
doc = Nokogiri::XML(open(fn))

tpl = Erubis::Eruby.new(open(File.join(File.dirname(__FILE__), 'template.rs.erb')).read)
svd = SVD.new(doc.root)

ACCESS_MAP = {
  'read-only'      => 'ro',
  'write-only'     => 'wo',
  'writeOnce'      => 'wo',
  'read-write'     => 'rw',
  'read-writeOnce' => 'rw',
}

puts tpl.evaluate(
  svd: svd,
  map_access: Proc.new do |i|
    name = ACCESS_MAP[i]
    raise RuntimeError.new("unknown access type #{i}") unless name
    if name != 'rw'
      ": #{name}"
    else
      ""
    end
  end
)
