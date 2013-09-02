require 'time'

module T
  class Data
    def initialize(path)
      @path = path
    end

    def entries
      @entries ||=
        if File.exists?(@path)
          File.readlines(@path).map { |line| Entry.new(*line.chomp.split(',')) }
        else
          []
        end
    end

    def start_entry(time)
      entries << Entry.new(time)
      write!
    end

    def stop_entry(entry, time)
      entry.stop = time
      write!
    end

    def write!
      File.open @path, 'w' do |f|
        entries.each do |entry|
          f.puts "#{entry.start},#{entry.stop}"
        end
      end
    end

    class Entry < Struct.new(:start, :stop)
      def minutes
        if stop.nil? || start.nil?
          0
        else
          ((Time.parse(stop) - Time.parse(start)) / 60).to_i
        end
      end

      def minutes_between(range_start, range_stop)
        if stop.nil? || start.nil?
          0
        else
          effective_start = [Time.parse(start), range_start].max
          effective_stop  = [Time.parse(stop),  range_stop ].min
          duration = effective_stop - effective_start
          return 0 if duration < 0
          (duration / 60).to_i
        end
      end
    end
  end
end
