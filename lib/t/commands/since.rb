require 't'
require 't/data'

module T
  module Commands
    class Since
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @time   = options.fetch(:time) { Time }
        @act    = options.fetch(:act) { T.activity_words }
      end

      def run
        data = Data.new(@file)
        total = self.total(data)
        if total == 0
          @stdout.puts "You have not #{@act.past_participle} #{period_description}."
        else
          @stdout.puts "You have #{@act.past_participle} for #{total} minutes #{period_description}."
        end
      end

      def total(data)
        now = @time.now.strftime(T::TIME_FORMAT)
        data.entries.each { |e| e.stop ||= now }.inject(0) { |sum, e| sum + e.minutes_between(range_start, range_stop) }
      end
    end
  end
end
