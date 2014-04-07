require 't'
require 't/data'

module T
  module Commands
    class PunchCard
      def initialize(options = {})
        @stdout           = options.fetch(:out) { $stdout }
        @file             = options.fetch(:file) { T::DATA_FILE }
        @zero_spark       = options.fetch(:zero) { " " }
        @sparks           = options.fetch(:sparks) { T::DEFAULT_SPARKS }
        @sparks           = [@sparks.first] + @sparks
        @terminal_columns = [options.fetch(:cols) { `tput cols`.to_i }, 80].max
      end

      def legend_type
        :week
      end

      def run
        data = Data.new(@file)
        bucket_count = @terminal_columns - LineHeaderWidth
        bucket_seconds = (7*86400.0) / bucket_count
        bucket_minutes = bucket_seconds / 60.0
        each_week(data) do |start_of_week, end_of_week, entries|
          buckets = (1..bucket_count).map { |n| [start_of_week + (n-1)*bucket_seconds, start_of_week + n*bucket_seconds] }
          bucket_segments = buckets.map { |bucket| entries.inject(0) { |n, e| n + e.minutes_between(*bucket) } }
          total = 0
          analysis = bucket_segments.map do |min|
            total += min
            spark(min, bucket_minutes)
          end.join
          @stdout.puts "#{line_header(start_of_week, end_of_week, total)}#{analysis}"
        end
      end

      private

      def spark(minutes, max_minutes)
        return @zero_spark if minutes == 0
        max_spark = @sparks.size - 1
        @sparks[[(max_spark * minutes.to_f / max_minutes).round, max_spark].min]
      end

      def each_week(data)
        if data.entries.any?
          earliest = data.earliest_date
          latest   = data.latest_time
          pending_entries = []
          future_entries = data.entries.select { |entry| entry.minutes > 0 }

          start_of_week = earliest - 86400 * earliest.wday
          while start_of_week < latest
            end_of_week = start_of_week + 7*86400
            current_entries, future_entries = future_entries.partition { |entry| entry.start_time < end_of_week }
            current_entries = pending_entries + current_entries

            yield start_of_week, end_of_week, current_entries

            pending_entries = current_entries.select { |entry| entry.stop_time > end_of_week }
            start_of_week = end_of_week
          end
        end
      end

      def line_header(from, to, total)
        self.class.line_header(from, to, total)
      end

      def self.line_header(from, to, total)
        "#{from.strftime(T::DATE_FORMAT)} - #{(to-1).strftime(T::DATE_FORMAT)}   #{'%4d' % total} min "
      end

      LineHeaderWidth = line_header(Time.now, Time.now, 0).size
    end
  end
end
