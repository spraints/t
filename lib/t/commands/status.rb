require 't'
require 't/data'

module T
  module Commands
    class Status
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
      end

      def legend_type
        :none
      end

      def run
        data = Data.new(@file)
        if data.entries.any? { |e| e.stop.nil? }
          @stdout.puts "WORKING"
        else
          @stdout.puts "NOT working"
        end
      end
    end
  end
end
