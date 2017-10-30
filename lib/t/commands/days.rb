require 't'
require 't/data'
require 't/util/week_grouping'

module T
  module Commands
    class Days
      def initialize(options = {})
        @stdout           = options.fetch(:out) { $stdout }
        @file             = options.fetch(:file) { T::DATA_FILE }
      end

      def legend_type
        :week
      end

      def run
        data = Data.new(@file)
        each_week(data) do |week_start, week_end, entries|
          cols = []
          cols << [week_start, week_end-1].map { |d| d.strftime(T::DATE_FORMAT) }.join(" - ")
          total = 0
          (0..6).each do |dow|
            day_start = week_start + dow * ONE_DAY
            day_end   = day_start + ONE_DAY
            day_minutes = entries.inject(0) { |n, e| n + e.minutes_between(day_start, day_end) }
            cols.push(day_minutes == 0 ? "   " : ("%3d" % day_minutes))
            total += day_minutes
          end
          cols << ("%5d" % total)
          @stdout.puts cols.join(" | ")
        end
      end

      private

      ONE_DAY = 60 * 60 * 24 # seconds

      def minutes_for_day(day_start, entries)
        day_end = day_start + ONE_DAY
        min = entries.inject(0) { |n, e| n + e.minutes_between(day_start, day_end) }
        min == 0 ? "   " : ("%3d" % min)
      end

      def each_week(data, &block)
        T::Util::WeekGrouping.new(data).each_week(&block)
      end
    end
  end
end
