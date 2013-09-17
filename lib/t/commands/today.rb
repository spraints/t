require 't'
require 't/commands/since'

module T
  module Commands
    class Today < Since
      def legend_type
        :day
      end

      def range_start
        @range_start ||= @time.now.to_date.to_time
      end

      def range_stop
        @range_stop ||= range_start + 86400
      end

      def period_description
        "today"
      end
    end
  end
end
