// Tremor Tracker [v1.0.0] — local edit: px-0 & explicit colour/bg motion (no bare .transition utility).

import React from "react"
import * as HoverCardPrimitives from "@radix-ui/react-hover-card"

import { cx } from "../../utils/cx"

interface TrackerBlockProps {
  key?: string | number
  color?: string
  tooltip?: string
  hoverEffect?: boolean
  defaultBackgroundColor?: string
}

const Block = ({
  color,
  tooltip,
  defaultBackgroundColor,
  hoverEffect,
}: TrackerBlockProps) => {
  const [open, setOpen] = React.useState(false)
  return (
    <HoverCardPrimitives.Root
      open={open}
      onOpenChange={setOpen}
      openDelay={0}
      closeDelay={0}
      tremor-id="tremor-raw"
    >
      <HoverCardPrimitives.Trigger onClick={() => setOpen(true)} asChild>
        <div className="size-full overflow-hidden px-0 transition-[color,background-color] first:rounded-l-[4px] first:pl-0 last:rounded-r-[4px] last:pr-0 sm:px-px">
          <div
            className={cx(
              "size-full rounded-[2px]",
              color || defaultBackgroundColor,
              hoverEffect ? "hover:opacity-75" : "",
            )}
          />
        </div>
      </HoverCardPrimitives.Trigger>
      {tooltip ? (
        <HoverCardPrimitives.Portal>
          <HoverCardPrimitives.Content
            sideOffset={10}
            side="top"
            align="center"
            avoidCollisions
            className={cx(
              "z-50 w-auto max-w-72 rounded-md px-2.5 py-1.5 text-xs shadow-lift",
              "text-ink-button-fg",
              "bg-ink-button",
            )}
          >
            {tooltip}
          </HoverCardPrimitives.Content>
        </HoverCardPrimitives.Portal>
      ) : null}
    </HoverCardPrimitives.Root>
  )
}

Block.displayName = "Block"

interface TrackerProps extends React.HTMLAttributes<HTMLDivElement> {
  data: TrackerBlockProps[]
  defaultBackgroundColor?: string
  hoverEffect?: boolean
}

const Tracker = React.forwardRef<HTMLDivElement, TrackerProps>(
  (
    {
      data = [],
      defaultBackgroundColor = "bg-track",
      className,
      hoverEffect,
      ...props
    },
    forwardedRef,
  ) => {
    return (
      <div
        ref={forwardedRef}
        className={cx("group flex h-7 w-full items-center gap-0.5", className)}
        {...props}
      >
        {data.map((props, index) => (
          <Block
            key={props.key ?? index}
            defaultBackgroundColor={defaultBackgroundColor}
            hoverEffect={hoverEffect}
            {...props}
          />
        ))}
      </div>
    )
  },
)

Tracker.displayName = "Tracker"

export { Tracker, type TrackerBlockProps }
