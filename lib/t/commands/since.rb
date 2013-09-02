require 't'
require 't/data'

module T
  module Commands
    class Since
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @time   = options.fetch(:time) { Time }
      end

      def run
        data = Data.new(@file)
        now = @time.now.strftime(T::TIME_FORMAT)
        total = data.entries.each { |e| e.stop ||= now }.inject(0) { |sum, e| sum + e.minutes_between(range_start, range_stop) }
        if total == 0
          @stdout.puts "You have not worked #{period_description}."
        else
          @stdout.puts "You have worked for #{total} minutes #{period_description}."
        end
      end
    end
  end
end
