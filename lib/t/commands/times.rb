require 't'
require 't/data'
require 't/util/week_grouping'

module T
  module Commands
    class Times
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

          entry_starts = entries.map(&:start_time)
          start_per_day = (0..6).map { |dow|
            day_start = week_start + dow * ONE_DAY
            day_end   = day_start + ONE_DAY
            t = entry_starts.select { |t| t >= day_start && t < day_end }.min
            t && t.strftime("%H:%M")
          }
          current_week << start_per_day
          current_month << start_per_day
          current_year << start_per_day

          show current_week
        end
        show current_month
        show current_year
      end

      private

      ONE_DAY = 60 * 60 * 24 # seconds

      def in?(period, day)
        period && period.include?(day)
      end

      def show(period)
        if period
          days = period.days.map { |t| t.nil? ? "     " : t }.join(" | ")
          @stdout.printf "%-23s || %s |\n", period.label, days
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

        def <<(starts_per_day)
          @starts_per_day = starts_per_day
        end

        def days
          @starts_per_day
        end
      end

      class AccumulateStarts
        def <<(starts_per_day)
          @starts_per_day ||= [nil]*7
          @starts_per_day = @starts_per_day.zip(starts_per_day).map { |a,b| a.nil? ? b : (b.nil? ? a : (a < b ? a : b)) }
        end

        def days
          @starts_per_day
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
          (@accum ||= AccumulateStarts.new) << x
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
          (@accum ||= AccumulateStarts.new) << x
        end

        def days
          @accum.days
        end
      end
    end
  end
end
