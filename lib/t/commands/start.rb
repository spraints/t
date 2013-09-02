require 't'
require 't/data'

module T
  module Commands
    class Start
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @time   = options.fetch(:time) { Time }
      end

      def run
        data = Data.new(@file)
        if entry = data.entries.detect { |e| e.stop.nil? }
          @stdout.puts "You already started working, at #{entry.start}!"
        else
          data.start_entry(@time.now.strftime("%Y-%m-%d %H:%M"))
          @stdout.puts "Starting work."
        end
      end
    end
  end
end
