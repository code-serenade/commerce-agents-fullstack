import type { ReactNode } from "react"
import {
  Card as CoreCard,
  CardAction as CoreCardAction,
  CardContent as CoreCardContent,
  CardDescription as CoreCardDescription,
  CardFooter as CoreCardFooter,
  CardHeader as CoreCardHeader,
  CardTitle as CoreCardTitle,
} from "@workspace/ui-core/components/card"

export interface CardProps {
  children: ReactNode
}

export interface CardHeaderProps {
  children: ReactNode
}

export interface CardFooterProps {
  children: ReactNode
}

export interface CardTitleProps {
  children: ReactNode
}

export interface CardDescriptionProps {
  children: ReactNode
}

export interface CardActionProps {
  children: ReactNode
}

export interface CardContentProps {
  children: ReactNode
}

export function Card({ children }: CardProps) {
  return <CoreCard>{children}</CoreCard>
}

export function CardHeader({ children }: CardHeaderProps) {
  return <CoreCardHeader>{children}</CoreCardHeader>
}

export function CardFooter({ children }: CardFooterProps) {
  return <CoreCardFooter>{children}</CoreCardFooter>
}

export function CardTitle({ children }: CardTitleProps) {
  return <CoreCardTitle>{children}</CoreCardTitle>
}

export function CardDescription({ children }: CardDescriptionProps) {
  return <CoreCardDescription>{children}</CoreCardDescription>
}

export function CardAction({ children }: CardActionProps) {
  return <CoreCardAction>{children}</CoreCardAction>
}

export function CardContent({ children }: CardContentProps) {
  return <CoreCardContent>{children}</CoreCardContent>
}
