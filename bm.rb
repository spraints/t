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
    res << pe(entry) if entry
  end
  res << pe(data) if data
  res
end

F = {:mon => 1, :day => 2, :hr => 3, :min => 4}
def pe(data)
  args = [nil, nil, nil, nil, nil, 0, nil]
  while data
    c, data = data
    case c
    when :tz
      _, data = data
      _, data = data
      h2, data = data
      h1, data = data
      sign, data = data
      tz = (D.fetch(h1)*10 + D.fetch(h2)) * 3600
      tz = -tz if sign == "-"
      args[6] = tz
    when :year
      y4, data = data
      y3, data = data
      y2, data = data
      y1, data = data
      args[0] = D.fetch(y1)*1000 + D.fetch(y2)*100 + D.fetch(y3)*10 + D.fetch(y4)
    when :mon, :day, :hr, :min
      d2, data = data
      d1, data = data
      args[F.fetch(c)] = D.fetch(d1)*10 + D.fetch(d2)
    else
      raise "Unexpected #{c.inspect} (#{data.inspect}) (#{args.inspect})"
    end
  end
  Time.new(*args)
end

DIGITS = %w[0 1 2 3 4 5 6 7 8 9]
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

Init = ->(c, _) do
  case c
  when *DIGITS
    [Year, c, nil]
  else
    [Init, nil, nil]
  end
end

class DigitConsumer
  def initialize(label, len, next_char, next_state)
    @label = label
    @len = len
    @next_char = next_char
    @next_state = next_state
  end

  def call(c, data)
    case c
    when *DIGITS
      [self, [c, data], nil]
    when @next_char
      [@next_state, [@label, data], nil]
      #if data.first.size != @len
      #  raise "error: #{@label} expected #{@len} digits, got #{data.first.size} (#{data.inspect})"
      #else
        #(prog = data[1] || []) << [@label, data[0]]
        #[@next_state, [[], prog], nil]
      #end
    else
      raise "error: #{@label} expected digit or #{@next_char.inspect}, got #{c.inspect} (#{data.inspect})"
    end
  end
end

Minute = ->(c, data) do
  case c
  when *DIGITS
    [Minute, [c, data], nil]
  when " "
    [TZPrefix, [:min, data], nil]
  when "\n"
    [Init, nil, [:min, data]]
  #when " ", "\n"
    #if data.first.size != 2
    #  raise "error: min expected 2 digits, got #{data.first.size} (#{data.inspect})"
    #else
      #(prog = data[1] || []) << [:min, data[0]]
      #[c == " " ? TZPrefix : FinishEntry, [[], prog], nil]
    #end
  else
    raise "error: min expected digit or #{" ".inspect} or #{"\n".inspect}, got #{c.inspect} (#{data.inspect})"
  end
end

FinishEntry = ->(c, data) do
  s, d, _ = Init.call(c, nil)
  [s, d, data]
end

TZ = DigitConsumer.new(:tz, 5, "\n", FinishEntry)
TZPrefix = ->(c, data) do
  case c
  when "+", "-"
    [TZ, [c, data], nil]
  else
    raise "error: tz expected + or -, got #{c.inspect} (#{data.inspect})"
  end
end
Hour = DigitConsumer.new(:hr, 2, ":", Minute)
Day = DigitConsumer.new(:day, 2, " ", Hour)
Month = DigitConsumer.new(:mon, 2, "-", Day)
Year = DigitConsumer.new(:year, 4, "-", Month)

def mkio
  #return StringIO.new(EXAMPLE)
  f = Tempfile.new('t-benchmarks')
  f.write(EXAMPLE)
  f.rewind
  f
end

control = candidate = nil
candidateio = mkio
controlio = mkio

Benchmark.bm(40) do |x|
  x.report("Array append") { (100*N).times { arr = []; 5.times { |i| arr << i } } }
  x.report("Hash append") { (100*N).times { hash = {}; [:a, :b, :c, :d, :e].each { |x| hash[x] = x } } }
  x.report("Time.parse") { controlio.rewind; control = controlio.readlines.map { |l| Time.parse(l) } }
  x.report("tp") { candidateio.rewind; candidate = tp(candidateio) }
end

p [control.first, control.last, control.size]
p [candidate.first, candidate.last, candidate.size]
