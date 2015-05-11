require 'csv'

require 't'
require 't/data'

module T
  module Commands
    class CSV
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
      end

      def legend_type
        :none
      end

      def run
        data = Data.new(@file)
        csv = ::CSV.new(@stdout)
        if data.entries.any?
          earliest = data.earliest_date
          latest   = data.latest_time
          start_of_week = earliest - 86400 * earliest.wday
          while start_of_week < latest
            end_of_week = start_of_week + 7*86400
            days = (1..7).map { |d| [start_of_week + (d-1)*86400, start_of_week + d*86400] }
            day_segments = days.map { |day| data.entries.map { |e| e.minutes_between(*day) }.select { |x| x > 0 } }
            segments = day_segments.flatten
            total = segments.inject(0, &:+)

            csv << [start_of_week.strftime("%Y-%m-%d"), total]
            
            start_of_week = end_of_week
          end
        end
      end
    end
  end
end
