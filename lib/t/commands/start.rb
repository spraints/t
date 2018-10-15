require 't'
require 't/data'

module T
  module Commands
    class Start
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
        if entry = data.entries.detect { |e| e.stop.nil? }
          @stdout.printf "You already started %s, %d minutes ago!\n",
            @act.present_participle, (@time.now - Time.parse(entry.start)) / 60
        else
          data.start_entry(@time.now.strftime(T::TIME_FORMAT))
          @stdout.puts "Starting #{@act.noun}."
        end
      end
    end
  end
end
