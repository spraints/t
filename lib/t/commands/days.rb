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
        current_month = nil
        current_year = nil
        each_week(data) do |week_start, week_end, entries|
          unless in?(current_year, week_start)
            show current_year
            current_year = Year.new(week_start)
          end
          unless in?(current_month, week_start)
            show current_month
            current_month = Month.new(week_start)
          end
          current_week = Week.new(week_start)

          min_per_day = (0..6).map { |dow|
            day_start = week_start + dow * ONE_DAY
            day_end   = day_start + ONE_DAY
            entries.inject(0) { |n, e| n + e.minutes_between(day_start, day_end) }
          }
          current_week << min_per_day
          current_month << min_per_day
          current_year << min_per_day

          show current_week
        end
        show current_month
        show current_year
#          cols = []
#          cols << [week_start, week_end-1].map { |d| d.strftime(T::DATE_FORMAT) }.join(" - ")
#          total = 0
#          (0..6).each do |dow|
#            day_minutes = entries.inject(0) { |n, e| n + e.minutes_between(day_start, day_end) }
#            cols.push(day_minutes == 0 ? "   " : ("%3d" % day_minutes))
#            total += day_minutes
#          end
#          cols << ("%5d" % total)
#          @stdout.puts cols.join(" | ")
      end

      private

      ONE_DAY = 60 * 60 * 24 # seconds

      def in?(period, day)
        period && period.include?(day)
      end

      def show(period)
        if period
          total = 0
          days = period.days.map { |min| total += min; min == 0 ? "     " : ("%5d" % min) }.join(" | ")
          @stdout.printf "%-23s || %s || %6d\n", period.label, days, total
        end
      end

      def each_week(data, &block)
        T::Util::WeekGrouping.new(data).each_week(&block)
      end

      class Week
        def initialize(day)
          @start = day
        end

        def label
          "%s - %s" % [@start.strftime(T::DATE_FORMAT), (@start + 6 * ONE_DAY).strftime(T::DATE_FORMAT)]
        end

        def <<(min_per_day)
          @min_per_day = min_per_day
        end

        def days
          @min_per_day
        end
      end

      class AccumulateMinutes
        def <<(min_per_day)
          @min_per_day ||= [0]*7
          @min_per_day = @min_per_day.zip(min_per_day).map { |a,b| a + b }
        end

        def days
          @min_per_day
        end
      end

      class Month
        def initialize(day)
          @year = day.year
          @month = day.month
        end

        def label
          "%4d-%02d" % [@year, @month]
        end

        def include?(day)
          @year == day.year &&
            @month == day.month
        end

        def <<(x)
          (@accum ||= AccumulateMinutes.new) << x
        end

        def days
          @accum.days
        end
      end

      class Year
        def initialize(day)
          @year = day.year
        end

        def label
          "%4d" % @year
        end

        def include?(day)
          @year == day.year
        end

        def <<(x)
          (@accum ||= AccumulateMinutes.new) << x
        end

        def days
          @accum.days
        end
      end
    end
  end
end
