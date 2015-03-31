require 't'
require 't/data'

module T
  module Commands
    class Stop
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @time   = options.fetch(:time) { Time }
        @act    = options.fetch(:act) { T.activity_words }
      end

      def legend_type
        :none
      end

      def run
        data = Data.new(@file)
        started_entries = data.entries.select { |e| e.stop.nil? }
        case started_entries.size
        when 1
          entry = started_entries.first
          data.stop_entry(entry, @time.now.strftime(T::TIME_FORMAT))
          @stdout.puts "You just #{@act.past_participle} for #{entry.minutes} minutes."
        when 0
          @stdout.puts "You haven't started #{@act.present_participle} yet!"
        else
          @stdout.puts "Your file has more than one session started. Please `t edit` to fix it."
        end
      end
    end
  end
end
