# This benchmark demonstrates that a bespoke parser is faster than a
# generic one, when we know exactly what the expected format is.
#
# It also shows that building an array with << is faster than building
# a hash with []=.

require "benchmark"
require "pp"
require "stringio"
require "tempfile"
require "time"

USE_STRINGIO = ARGV[0] == "--stringio"
USE_ARG = !ARGV[0].nil? && File.exist?(ARGV[0])

N = 10000
EXAMPLE = "2020-01-01 12:34 -0700\n2020-02-02 12:34\n" * N

def tp(io)
  res = []
  state = Init
  data = nil
  io.each_char do |c|
  #io.read.each_char do |c|
  #until io.eof?
    #c = io.read(1)
    state, data, entry = state.call(c, data)
    res << entry if entry
  end
  res << Time.new(*data) if data
  res
end

D = {
  "0" => 0,
  "1" => 1,
  "2" => 2,
  "3" => 3,
  "4" => 4,
  "5" => 5,
  "6" => 6,
  "7" => 7,
  "8" => 8,
  "9" => 9,
}

DIGITS = D.keys

F = {
  :year => 0,
  :mon => 1,
  :day => 2,
  :hr => 3,
  :min => 4,
  :tz => 6,
}

Init = ->(c, _) do
  case c
  when *DIGITS
    [Year, [D.fetch(c), 0, 0, 0, 0, 0, nil], nil]
  else
    [Init, nil, nil]
  end
end

class DigitConsumer
  def initialize(label, len, next_char, next_state)
    @label = label
    @index = F.fetch(label)
    @len = len
    @next_char = next_char
    @next_state = next_state
  end

  def call(c, data)
    case c
    when *DIGITS
      data[@index] = 10*data[@index] + D.fetch(c)
      [self, data, nil]
    when @next_char
      [@next_state, data, nil]
    else
      raise "error: #{@label} expected digit or #{@next_char.inspect}, got #{c.inspect} (#{data.inspect})"
    end
  end
end

MinuteIndex = F.fetch(:min)

Minute = ->(c, data) do
  case c
  when *DIGITS
    data[MinuteIndex] = 10*data[MinuteIndex] + D.fetch(c)
    [Minute, data, nil]
  when " "
    [TZPrefix, data, nil]
  when "\n", ","
    [Init, nil, Time.new(*data)]
  else
    raise "error: min expected digit or #{" ".inspect} or #{"\n".inspect}, got #{c.inspect} (#{data.inspect})"
  end
end

TZIndex = F.fetch(:tz)

TZPrefix = ->(c, data) do
  case c
  when "+"
    data[TZIndex] = 0
    [PosTZ, data, nil]
  when "-"
    data[TZIndex] = 0
    [NegTZ, data, nil]
  else
    raise "error: tz expected + or -, got #{c.inspect} (#{data.inspect})"
  end
end

class TZ
  def self.new(sign)
    stack = TZFinish.new(sign)
    stack = TZDigit.new(10*60, stack)
    stack = TZDigit.new(60*60, stack)
    stack = TZDigit.new(10*60*60, stack)
  end

  class TZFinish
    def initialize(sign)
      @sign = sign
    end

    def call(c, data)
      data[TZIndex] += D.fetch(c) * 60
      data[TZIndex] *= @sign
      [Init, nil, Time.new(*data)]
    end
  end

  class TZDigit
    def initialize(fact, stack)
      @fact = fact
      @stack = stack
    end

    def call(c, data)
      data[TZIndex] += D.fetch(c) * @fact
      [@stack, data, nil]
    end
  end
end

NegTZ = TZ.new(-1)
PosTZ = TZ.new(1)
Hour = DigitConsumer.new(:hr, 2, ":", Minute)
Day = DigitConsumer.new(:day, 2, " ", Hour)
Month = DigitConsumer.new(:mon, 2, "-", Day)
Year = DigitConsumer.new(:year, 4, "-", Month)

$first = true
def mkio
  if USE_STRINGIO
    puts "Using stringio" if $first
    $first = false
    return StringIO.new(EXAMPLE)
  end
  if USE_ARG
    puts "Using #{ARGV[0]}" if $first
    $first = false
    return File.open(ARGV[0])
  end
  puts "Using tempfile"
  f = Tempfile.new('t-benchmarks')
  f.write(EXAMPLE)
  f.rewind
  f
end

control = candidate = nil
candidateio = mkio
controlio = mkio

Benchmark.bmbm(40) do |x|
  x.report("Time.parse") do
    controlio.rewind
    control = controlio.readlines.flat_map { |l| l.split(",").map { |t| Time.parse(t) } }
  end
  x.report("tp") do
    candidateio.rewind
    candidate = tp(candidateio)
  end
end

p [control.first, control.last, control.size]
p [candidate.first, candidate.last, candidate.size]
