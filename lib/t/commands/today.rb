require 't'
require 't/commands/since'

module T
  module Commands
    class Today < Since
      def range_start
        @range_start ||= Time.parse(@time.now.strftime(T::DATE_FORMAT))
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
