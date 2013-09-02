require 't'
require 't/data'

module T
  module Commands
    class All
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @tail   = options.fetch(:tail) { (1..5).to_a.map { |d| "#{d*8}h=#{d*8*60}m" }.join(' ') }
      end

      def run
        data = Data.new(@file)
        if data.entries.any?
          earliest = Time.parse(data.entries.map(&:start).compact.min)
          latest   = Time.parse(data.entries.map(&:stop).compact.max)
          earliest = Time.parse(earliest.strftime(T::DATE_FORMAT))
          start_of_week = earliest - 86400 * earliest.wday
          while start_of_week < latest
            end_of_week = start_of_week + 7*86400
            total = data.entries.inject(0) { |sum, e| sum + e.minutes_between(start_of_week, end_of_week) }
            @stdout.puts "#{start_of_week.strftime(T::DATE_FORMAT)} - #{(end_of_week-1).strftime(T::DATE_FORMAT)}   #{'%4d' % total} min"
            start_of_week = end_of_week
          end
        end
        @stdout.puts @tail if @tail
      end
    end
  end
end
