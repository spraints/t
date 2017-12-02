require 't'
require 't/data'

module T
  module Commands
    class Status
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @act    = options.fetch(:act) { T.activity_words }
        @week   = options.fetch(:week_calculator)
      end

      def legend_type
        :none
      end

      def run(*args)
        data = Data.new(@file)
        if args.include?("--with-week")
          suffix = " (#{@week.total(data)})"
        end
        if data.entries.any? { |e| e.stop.nil? }
          @stdout.puts "#{@act.present_participle.upcase}#{suffix}"
        else
          @stdout.puts "NOT #{@act.present_participle}#{suffix}"
        end
      end
    end
  end
end
