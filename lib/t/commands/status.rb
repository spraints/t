require 't'
require 't/data'
require 't/commands/week'

module T
  module Commands
    class Status
      def initialize(options = {})
        @stdout = options.fetch(:out) { $stdout }
        @file   = options.fetch(:file) { T::DATA_FILE }
        @act    = options.fetch(:act) { T.activity_words }
        @week   = T::Commands::Week.new(options)
      end

      def legend_type
        :none
      end

      def run(*args)
        data = Data.new(@file)
        suffix = self.suffix(*args)
        if data.entries.any? { |e| e.stop.nil? }
          @stdout.puts "#{@act.present_participle.upcase}#{suffix}"
        else
          @stdout.puts "NOT #{@act.present_participle}#{suffix}"
        end
      end

      def suffix(*args)
        if args.include?("--with-week")
          " (#{@week.total})"
        else
          ""
        end
      end
    end
  end
end
