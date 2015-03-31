require 't'
require 't/data'

module T
  module Commands
    class Status
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @act    = options.fetch(:act) { T.activity_words }
      end

      def legend_type
        :none
      end

      def run
        data = Data.new(@file)
        if data.entries.any? { |e| e.stop.nil? }
          @stdout.puts "#{@act.present_participle.upcase}"
        else
          @stdout.puts "NOT #{@act.present_participle}"
        end
      end
    end
  end
end
