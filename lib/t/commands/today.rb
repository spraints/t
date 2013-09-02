require 't'
require 't/data'

module T
  module Commands
    class Today
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @time   = options.fetch(:time) { Time }
      end

      def run
        data = Data.new(@file)
        now = @time.now.strftime("%Y-%m-%d %H:%M")
        today_start = Time.parse(@time.now.strftime("%Y-%m-%d"))
        today_stop  = today_start + 86400
        total = data.entries.each { |e| e.stop ||= now }.inject(0) { |sum, e| sum + e.minutes_between(today_start, today_stop) }
        if total == 0
          @stdout.puts "You have not worked today."
        else
          @stdout.puts "You have worked for #{total} minutes today."
        end
      end
    end
  end
end
