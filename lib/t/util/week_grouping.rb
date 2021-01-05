module T
  module Util
    class WeekGrouping
      def initialize(data)
        @data = data
      end

      attr_reader :data

      def each_week
        if data.entries.any?
          earliest = data.earliest_date
          latest   = data.latest_time
          pending_entries = []
          future_entries = data.entries.select { |entry| entry.minutes > 0 }

          start_of_week = earliest - 86400 * earliest.wday
          while start_of_week < latest
            end_of_week = add_week(start_of_week)
            current_entries, future_entries = future_entries.partition { |entry| entry.start_time < end_of_week }
            current_entries = pending_entries + current_entries

            yield start_of_week, end_of_week, current_entries

            pending_entries = current_entries.select { |entry| entry.stop_time > end_of_week }
            start_of_week = end_of_week
          end
        end
      end

      def add_week(time)
        next_time = time + 7*86400
        utc_delta = time.utc_offset - next_time.utc_offset
        next_time + utc_delta
      end
    end
  end
end
