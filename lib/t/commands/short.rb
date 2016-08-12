require 't'
require 't/data'
require 't/util/week_grouping'

module T
  module Commands
    class Short
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @step   = options.fetch(:step) { 100 }
      end

      def run(limit = "15")
        limit = limit.to_i
        each_week do |start, stop, entries|
          short = entries.select { |e| e.minutes <= limit }
          short_minutes, all_minutes = minutes(short), minutes(entries)
          @stdout.printf "%s - %s   %3d/%3d  (%5d/%5d minutes)  %s\n",
            df(start), df(stop - 1),
            short.size, entries.size,
            short_minutes, all_minutes,
            graph(short_minutes, all_minutes)
        end
      end

      def legend_type
        :week
      end

      private

      def graph(short_minutes, all_minutes)
        all_bars = 1 + (all_minutes - 1) / @step
        short_bars = 1 + (short_minutes - 1) / @step
        long_bars = all_bars - short_bars
        "."*short_bars + "|"*long_bars
      end

      def each_week(&block)
        data = Data.new(@file)
        T::Util::WeekGrouping.new(data).each_week(&block)
      end

      def df(date)
        date.strftime(T::DATE_FORMAT)
      end

      def minutes(entries)
        entries.inject(0) { |min, e| min + e.minutes }
      end
    end
  end
end
