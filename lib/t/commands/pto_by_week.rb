require 't'
require 't/data'
require 't/util/week_grouping'

module T
  module Commands
    class Pto
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
      end

      def legend_type
        :week
      end

      def run(full_week=nil)
        full_week = full_week.to_i
        full_week = 5*8*60 if full_week < 1

        each_week do |week_start, week_end, entries|
          minutes_on = entries.inject(0) { |total, entry| total + entry.minutes_between(week_start, week_end) }
          minutes_off = [full_week - minutes_on, 0].max
          @stdout.printf "%s work=%4d pto=%4d\n",
            week_start.strftime(T::DATE_FORMAT), minutes_on, minutes_off
        end
      end

      def each_week(&block)
        data = Data.new(@file)
        T::Util::WeekGrouping.new(data).each_week(&block)
      end
    end
  end
end
