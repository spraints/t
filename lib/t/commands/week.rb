require 't'
require 't/commands/since'

module T
  module Commands
    class Week < Since
      def legend_type
        :week
      end

      def range_start
        @range_start ||= @time.now.to_date.to_time - 86400*@time.now.wday
      end

      def range_stop
        @range_stop ||= range_start + 7*86400
      end

      def period_description
        "since #{range_start.strftime(T::DATE_FORMAT)}"
      end
    end
  end
end
