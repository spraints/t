require 'time'

module T
  class Data
    def initialize(path_or_entries)
      if path_or_entries.is_a?(Array)
        @entries = path_or_entries
      else
        @path = path_or_entries
      end
    end

    def entries
      @entries ||=
        if File.exists?(@path)
          File.readlines(@path).grep_v(/^#/).map { |line| Entry.new(*line.chomp.split(',')) }
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

    def earliest_time
      Time.parse(entries.map(&:start).compact.min)
    end

    def latest_time
      Time.parse(entries.map(&:stop).compact.max)
    end

    def earliest_date
      earliest_time.to_date.to_time
    end

    def latest_date
      latest_time.to_date.to_time
    end

    class Entry < Struct.new(:start, :stop)
      @@parsed_times = {}

      def minutes
        if stop.nil? || start.nil?
          0
        else
          ((stop_time - start_time) / 60).to_i
        end
      end

      def minutes_between(range_start, range_stop)
        if stop.nil? || start.nil?
          0
        else
          effective_start = [start_time, range_start].max
          effective_stop  = [stop_time,  range_stop ].min
          duration = effective_stop - effective_start
          return 0 if duration < 0
          (duration / 60).to_i
        end
      end

      def start_time
        start && @@parsed_times[start] ||= parse_time(start)
      end

      def stop_time
        stop && @@parsed_times[stop] ||= parse_time(stop)
      end

      private

      def parse_time(s)
        Time.parse(s)
      rescue
        p s ; raise
      end
    end
  end
end
