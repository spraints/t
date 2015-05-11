require "t/commands/csv"

module T
  module Commands
    class SVG
      def initialize(options = {})
        @options   = options
        @stdout    = options.fetch(:out) { $stdout }
        @csv_class = options.fetch(:csv_class) { CSV }
      end

      def legend_type
        :none
      end

      def run
        r,w = IO.pipe

        clic_pid = spawn "clic-line",
          "--xkey", "week",
          "--ykey", "minutes",
          "--time-series",
          "--time-format", "%Y-%m-%d",
          :in => r, :out => @stdout
        r.close

        @csv_class.new(@options.merge(:out => w)).run
        w.close

        Process.waitpid(clic_pid)
      end
    end
  end
end
