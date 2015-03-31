require 't'
require 't/data'
require 't/util/week_grouping'

module T
  module Commands
    class Pto
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @act    = options.fetch(:act) { T.activity_words }
      end

      def legend_type
        :week
      end

      def run(full_week=nil)
        full_week = full_week.to_i
        full_week = 5*8*60 if full_week < 1

        year_totals = Hash.new(0)

        each_week do |week_start, week_end, entries|
          minutes_on = entries.inject(0) { |total, entry| total + entry.minutes_between(week_start, week_end) }
          next if minutes_on == 0
          minutes_off = [full_week - minutes_on, 0].max
          year_totals[week_start.year] += minutes_off
          @stdout.printf "%s #{@act.noun}=%4d pto=%4d\n",
            week_start.strftime(T::DATE_FORMAT), minutes_on, minutes_off
        end

        if year_totals.any?
          @stdout.puts
        end
        year_totals.sort_by(&:first).each do |year, total|
          @stdout.printf "%s total_pto=%5d days=%3d\n", year, total, total/60/8
        end
      end

      def each_week(&block)
        data = Data.new(@file)
        T::Util::WeekGrouping.new(data).each_week(&block)
      end
    end
  end
end
