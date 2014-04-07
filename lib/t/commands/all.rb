require 't'
require 't/data'

module T
  module Commands
    class All
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @sparks = options.fetch(:sparks) { T::DEFAULT_SPARKS }
      end

      def legend_type
        :week
      end

      def run
        data = Data.new(@file)
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
            analysis =
              if segments.size > 1
                mean = total / segments.size
                max = segments.max
                stddev = Math.sqrt(segments.inject(0.0) { |accum, segment| accum + (segment - mean)**2 } / (segments.size - 1))
                spark = day_segments.select { |x| x.size > 0 }.map { |day_segment| day_segment.map { |segment| @sparks[segment * @sparks.size / max] || @sparks.last }.join('') }.join('  ')
                ' %4d segments  min/avg/max/stddev=%3d/%3d/%3d/%3d  %s' % [segments.size, segments.min, mean, max, stddev, spark]
              else
                ''
              end
            @stdout.puts "#{start_of_week.strftime(T::DATE_FORMAT)} - #{(end_of_week-1).strftime(T::DATE_FORMAT)}   #{'%4d' % total} min#{analysis}"
            start_of_week = end_of_week
          end
        end
      end
    end
  end
end
